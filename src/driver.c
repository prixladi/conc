#include <sys/stat.h>
#include <sys/types.h>
#include <stdlib.h>
#include <dirent.h>
#include <stdio.h>
#include <stdbool.h>
#include <unistd.h>
#include <signal.h>
#include <dirent.h>

#include "utils/string.h"
#include "utils/vector.h"
#include "utils/log.h"
#include "utils/time.h"
#include "utils/fs.h"

#include "driver.h"
#include "process.h"

static const char *root_projects_dir = "./projects";
static const char *log_file_name = "log";
static const char *meta_file_name = "meta";

struct service_process_info
{
    int pid;
    time_t c_time;
};

static int ensure_project_dir_exists(const char *proj_name);
static int ensure_service_dir_exists(const char *proj_name, const char *serv_name);

static int write_service_meta_file(const char *proj_name, const char *serv_name, struct service_process_info info);
static bool try_parse_service_meta_file(const char *proj_name, const char *serv_name, struct service_process_info *info);

static FILE *open_service_meta_file(const char *proj_name, const char *serv_name, const char *mode);
static FILE *open_project_meta_file(const char *proj_name, const char *mode);

static char *get_project_dir_path(const char *proj_name);
static char *get_project_meta_file_path(const char *proj_name);
static char *get_service_dir_path(const char *proj_name, const char *serv_name);
static char *get_service_meta_file_path(const char *proj_name, const char *serv_name);
static char *get_service_log_file_path(const char *proj_name, const char *serv_name);

static int remove_file_f(char *path);
static int remove_dir_f(char *path);

static int get_running_service_pid(const char *proj_name, const char *serv_name);
static bool try_get_pid_info(int pid, struct stat *sts);
static int kill_pid(int pid);

enum d_result
driver_mount(void)
{
    mkdir(root_projects_dir, S_IRWXU | S_IRWXG | S_IRWXO);

    DIR *dir = opendir(root_projects_dir);
    if (dir == NULL)
    {
        log_critical("Driver root project dir init failed\n");
        return D_FS_ERROR;
    }

    closedir(dir);

    log_info("Driver mounted\n");
    return D_OK;
}

enum d_result
driver_unmount(void)
{
    log_info("Driver unmounted\n");
    return D_NO_ACTION;
}

char **
d_get_all_stored_settings(void)
{
    char **settings_vec = vec_create(char *);

    DIR *projects_dir = opendir(root_projects_dir);
    if (!projects_dir)
        return settings_vec;

    struct dirent *entry;
    while ((entry = readdir(projects_dir)) != NULL)
    {
        if (strncmp(entry->d_name, ".", 1) == 0 || strncmp(entry->d_name, "..", 2) == 0)
            continue;

        char *settings_file_path = STR_CONCAT(root_projects_dir, "/", entry->d_name, "/", meta_file_name);
        FILE *fp = fopen(settings_file_path, "r");
        if (!fp)
            log_error("Unable to load settings from '%s'\n", settings_file_path);
        else
        {
            char *content = get_file_content(fp);
            fclose(fp);
            vec_push(settings_vec, content);
        }
        free(settings_file_path);
    }
    closedir(projects_dir);

    return settings_vec;
}

enum d_result
d_project_init(const struct project_settings settings)
{
    ensure_project_dir_exists(settings.name);

    FILE *fp = open_project_meta_file(settings.name, "w");
    if (fp == NULL)
    {
        log_critical("Unable to open meta file for project. Project: '%s'", settings.name);
        return D_FS_ERROR;
    }

    char *stringified_settings = project_settings_stringify(settings);
    fprintf(fp, "%s", stringified_settings);
    fclose(fp);
    free(stringified_settings);

    for (size_t i = 0; i < vec_length(settings.services); i++)
        ensure_service_dir_exists(settings.name, settings.services[i].name);

    return D_OK;
}

enum d_result
d_project_remove(const struct project_settings settings)
{
    for (size_t i = 0; i < vec_length(settings.services); i++)
    {
        struct service_settings service = settings.services[i];
        remove_file_f(get_service_meta_file_path(settings.name, service.name));
        remove_file_f(get_service_log_file_path(settings.name, service.name));
        remove_dir_f(get_service_dir_path(settings.name, service.name));
    }

    remove_file_f(get_project_meta_file_path(settings.name));

    char *project_dir_path = get_project_dir_path(settings.name);
    rmdir(project_dir_path);

    bool delete_success = !dir_exists(project_dir_path);
    if (!delete_success)
        log_error("Unable to remove project directory '%s'\n", project_dir_path);

    free(project_dir_path);
    return delete_success ? D_OK : D_FS_ERROR;
}

enum d_result
d_service_info_get(const char *proj_name, const char *serv_name, struct d_service_info *info)
{
    int running_pid = get_running_service_pid(proj_name, serv_name);

    if (running_pid > 0)
    {
        info->status = D_RUNNING;
        info->pid = running_pid;
    }
    else
    {
        info->status = running_pid == 0 ? D_STOPPED : D_NONE;
        info->pid = -1;
    }

    return D_OK;
}

enum d_result
d_service_start(const char *proj_name, const struct service_settings service_settings)
{
    int running_pid = get_running_service_pid(proj_name, service_settings.name);
    if (running_pid > 0)
        return D_NO_ACTION;

    char *logfile_path = get_service_log_file_path(proj_name, service_settings.name);
    int pid = process_start(proj_name, service_settings, logfile_path);

    free(logfile_path);

    time_t c_time = 0;

    struct stat sts;
    if (try_get_pid_info(pid, &sts))
        c_time = sts.st_ctime;

    struct service_process_info info = {
        .pid = pid,
        .c_time = c_time,
    };

    if (write_service_meta_file(proj_name, service_settings.name, info) > 0)
        return D_FS_ERROR;

    return D_OK;
}

enum d_result
d_service_stop(const char *proj_name, const struct service_settings service_settings)
{
    int running_pid = get_running_service_pid(proj_name, service_settings.name);
    if (running_pid <= 0)
        return D_NO_ACTION;

    if (kill_pid(running_pid) > 0)
        return D_PROC_ERROR;

    return D_OK;
}

static int
get_running_service_pid(const char *proj_name, const char *serv_name)
{
    struct service_process_info info = { 0 };
    if (try_parse_service_meta_file(proj_name, serv_name, &info) == false)
        return -1;

    struct stat sts;
    if (try_get_pid_info(info.pid, &sts) == false)
        return 0;

    return info.pid;
}

static int
ensure_service_dir_exists(const char *proj_name, const char *serv_name)
{
    char *service_dir = STR_CONCAT(root_projects_dir, "/", proj_name, "/", serv_name);
    int result = mkdir(service_dir, S_IRWXU | S_IRWXG | S_IRWXO);
    free(service_dir);
    return result;
}

static int
ensure_project_dir_exists(const char *proj_name)
{
    char *project_dir = STR_CONCAT(root_projects_dir, "/", proj_name);
    int result = mkdir(project_dir, S_IRWXU | S_IRWXG | S_IRWXO);
    free(project_dir);
    return result;
}

static int
write_service_meta_file(const char *proj_name, const char *serv_name, struct service_process_info info)
{
    FILE *fp = open_service_meta_file(proj_name, serv_name, "w");
    if (fp == NULL)
    {
        log_error("Unable to open meta file for service. Project: '%s', service: %s", proj_name, serv_name);
        return 1;
    }

    fprintf(fp, "%d\n%ld", info.pid, info.c_time);
    fclose(fp);
    return 0;
}

#define MAX_META_LINE_LEN 1024 // This should be sufficient but probably should handle cases when it is not
static bool
try_parse_service_meta_file(const char *proj_name, const char *serv_name, struct service_process_info *info)
{
    FILE *fp = open_service_meta_file(proj_name, serv_name, "r");
    if (fp == NULL)
        return false;

    char buffer[MAX_META_LINE_LEN];

    int parsed = 0;
    if (fgets(buffer, MAX_META_LINE_LEN, fp))
    {
        buffer[strcspn(buffer, "\n")] = '\0';
        int pid = atoi(buffer);
        if (pid != 0)
        {
            info->pid = pid;
            parsed++;
        }
    }

    if (fgets(buffer, MAX_META_LINE_LEN, fp))
    {
        buffer[strcspn(buffer, "\n")] = '\0';
        time_t c_time = atoll(buffer);
        if (c_time != 0)
        {
            info->c_time = c_time;
            parsed++;
        }
    }

    fclose(fp);
    return parsed == 2;
}

static FILE *
open_project_meta_file(const char *proj_name, const char *modes)
{
    char *meta_file_path = get_project_meta_file_path(proj_name);
    FILE *fp = fopen(meta_file_path, modes);
    free(meta_file_path);

    return fp;
}

static FILE *
open_service_meta_file(const char *proj_name, const char *serv_name, const char *modes)
{
    char *meta_file_path = get_service_meta_file_path(proj_name, serv_name);
    FILE *fp = fopen(meta_file_path, modes);
    free(meta_file_path);
    return fp;
}

static char *
get_project_dir_path(const char *proj_name)
{
    return STR_CONCAT(root_projects_dir, "/", proj_name);
}

static char *
get_project_meta_file_path(const char *proj_name)
{
    return STR_CONCAT(root_projects_dir, "/", proj_name, "/", meta_file_name);
}

static char *
get_service_dir_path(const char *proj_name, const char *serv_name)
{
    return STR_CONCAT(root_projects_dir, "/", proj_name, "/", serv_name);
}

static char *
get_service_meta_file_path(const char *proj_name, const char *serv_name)
{
    return STR_CONCAT(root_projects_dir, "/", proj_name, "/", serv_name, "/", meta_file_name);
}

static char *
get_service_log_file_path(const char *proj_name, const char *serv_name)
{
    return STR_CONCAT(root_projects_dir, "/", proj_name, "/", serv_name, "/", log_file_name);
}

static int
remove_file_f(char *path)
{
    int res = remove(path);
    free(path);
    return res;
}

static int
remove_dir_f(char *path)
{
    int res = rmdir(path);
    free(path);
    return res;
}

static bool
try_get_pid_info(int pid, struct stat *sts)
{
    char *pid_string = int_to_str(pid);
    char *proc = STR_CONCAT("/proc/", pid_string);
    free(pid_string);

    int result = stat(proc, sts);
    free(proc);

    return result != -1;
}

static int
kill_pid(int pid)
{
    struct stat sts;
    if (try_get_pid_info(pid, &sts) == false)
        return 0;

    int attempt = 0;
    while (attempt < 10)
    {
        if (attempt)
            sleep_ms(50);

        kill(pid, attempt > 6 ? SIGKILL : SIGTERM);

        if (try_get_pid_info(pid, &sts) == false)
            return 0;

        attempt++;
    }

    return attempt;
}
