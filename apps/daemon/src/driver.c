#include <stdlib.h>
#include <dirent.h>
#include <stdio.h>
#include <stdbool.h>
#include <unistd.h>
#include <fcntl.h>
#include <signal.h>
#include <dirent.h>

#include <sys/wait.h>
#include <sys/stat.h>
#include <sys/types.h>

#include "utils/string.h"
#include "utils/vector.h"
#include "utils/log.h"
#include "utils/fs.h"
#include "utils/memory.h"

#include "driver.h"
#include "process.h"

static const char *root_projects_dir = "./projects";
static const char *logfile_name = "log";
static const char *meta_file_name = "meta";

struct service_process_info
{
    int pid;
    time_t c_time;
    time_t s_time;
};

static int ensure_project_dir_exists(const char *proj_name);
static int ensure_service_dir_exists(const char *proj_name, const char *serv_name);
static int ensure_service_logfile_exists(const char *proj_name, const char *serv_name);

static int write_service_meta_file(const char *proj_name, const char *serv_name, struct service_process_info info);
static bool try_parse_service_meta_file(const char *proj_name, const char *serv_name, struct service_process_info *info);

static FILE *open_service_meta_file(const char *proj_name, const char *serv_name, const char *mode);
static FILE *open_project_meta_file(const char *proj_name, const char *mode);

static char *get_project_dir_path(const char *proj_name);
static char *get_project_meta_file_path(const char *proj_name);
static char *get_service_dir_path(const char *proj_name, const char *serv_name);
static char *get_service_meta_file_path(const char *proj_name, const char *serv_name);
static char *get_service_logfile_path(const char *proj_name, const char *serv_name);

static struct service_process_info get_service_info(const char *proj_name, const char *serv_name);
static bool is_proccess_running(int pid);

static int remove_file_f(char *path);
static int remove_dir_f(char *path);

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

        scoped char *settings_file_path = str_printf("%s/%s/%s", root_projects_dir, entry->d_name, meta_file_name);
        FILE *fp = fopen(settings_file_path, "r");
        if (!fp)
        {
            log_error("Unable to load settings from '%s'\n", settings_file_path);
            continue;
        }

        char *content = get_file_content(fp);
        fclose(fp);
        vec_push(settings_vec, content);
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
        log_critical("Unable to open meta file for project '%s'\n", settings.name);
        return D_FS_ERROR;
    }

    scoped char *stringified_settings = project_settings_stringify(settings);
    fprintf(fp, "%s", stringified_settings);
    fclose(fp);

    for (size_t i = 0; i < vec_length(settings.services); i++)
    {
        ensure_service_dir_exists(settings.name, settings.services[i].name);
        ensure_service_logfile_exists(settings.name, settings.services[i].name);
    }

    return D_OK;
}

enum d_result
d_project_remove(const struct project_settings settings)
{
    for (size_t i = 0; i < vec_length(settings.services); i++)
    {
        struct service_settings service = settings.services[i];
        remove_file_f(get_service_meta_file_path(settings.name, service.name));
        remove_file_f(get_service_logfile_path(settings.name, service.name));
        remove_dir_f(get_service_dir_path(settings.name, service.name));
    }

    remove_file_f(get_project_meta_file_path(settings.name));

    scoped char *project_dir_path = get_project_dir_path(settings.name);
    rmdir(project_dir_path);

    bool delete_success = !dir_exists(project_dir_path);
    if (!delete_success)
        log_error("Unable to remove project directory '%s'\n", project_dir_path);

    return delete_success ? D_OK : D_FS_ERROR;
}

enum d_result
d_service_info_get(const char *proj_name, const char *serv_name, struct d_service_info *info)
{
    struct service_process_info process_info = get_service_info(proj_name, serv_name);

    if (process_info.pid > 0)
        info->status = D_RUNNING;
    else if (process_info.pid == 0)
        info->status = process_info.s_time == 0 ? D_EXITED : D_STOPPED;
    else
        info->status = D_NONE;

    info->pid = process_info.pid;
    info->start_time = process_info.c_time;
    info->stop_time = process_info.s_time;
    scoped char *log_path = get_service_logfile_path(proj_name, serv_name);
    info->logfile_path = realpath(log_path, NULL);

    return D_OK;
}

enum d_result
d_service_start(const struct project_settings project, const struct service_settings service_settings,
                const struct env_variable *env)
{
    struct service_process_info process_info = get_service_info(project.name, service_settings.name);
    if (process_info.pid > 0)
        return D_NO_ACTION;

    scoped char *logfile_path = get_service_logfile_path(project.name, service_settings.name);
    int pid = process_start(project, service_settings, env, logfile_path);

    time_t c_time = 0;
    
    struct stat sts;
    scoped char *proc = str_printf("/proc/%d", pid);
    // TODO: maybe i should just use UTC.now instead of this, idk
    if (stat(proc, &sts) == 0)
        c_time = sts.st_ctime;

    struct service_process_info info = {
        .pid = pid,
        .c_time = c_time,
    };

    if (write_service_meta_file(project.name, service_settings.name, info) > 0)
    {
        log_error("Unable to write service meta for service '%s' in project '%s'\n", service_settings.name, project.name);
        if (pid > 0)
            process_kill_pid_pg(pid);
        return D_FS_ERROR;
    }

    return D_OK;
}

enum d_result
d_service_stop(const char *proj_name, const struct service_settings service_settings)
{
    struct service_process_info process_info = get_service_info(proj_name, service_settings.name);
    if (process_info.pid <= 0)
        return D_NO_ACTION;

    log_debug("Stopping process '%s/%s - %d'\n", proj_name, service_settings.name, process_info.pid);

    if (process_kill_pid_pg(process_info.pid) > 0)
    {
        log_error("Unable to kill PID '%d'\n", process_info.pid);
        return D_PROC_ERROR;
    }

    process_info.s_time = time(NULL);

    if (write_service_meta_file(proj_name, service_settings.name, process_info) > 0)
    {
        log_error("Unable to write service meta for service '%s' in project '%s'\n", service_settings.name, proj_name);
        return D_FS_ERROR;
    }

    return D_OK;
}

void
d_service_info_free(struct d_service_info info)
{
    free(info.logfile_path);

    info.logfile_path = NULL;
}

static struct service_process_info
get_service_info(const char *proj_name, const char *serv_name)
{
    struct service_process_info info = { 0 };
    if (try_parse_service_meta_file(proj_name, serv_name, &info) == false)
    {
        info.pid = -1;
        info.c_time = 0;
        info.s_time = 0;
        return info;
    }

    if (info.pid > 0 && !is_proccess_running(info.pid))
        info.pid = 0;

    return info;
}

static int
ensure_project_dir_exists(const char *proj_name)
{
    scoped char *project_dir = get_project_dir_path(proj_name);
    return mkdir(project_dir, S_IRWXU | S_IRWXG | S_IRWXO);
}

static int
ensure_service_dir_exists(const char *proj_name, const char *serv_name)
{
    scoped char *service_dir = get_service_dir_path(proj_name, serv_name);
    return mkdir(service_dir, S_IRWXU | S_IRWXG | S_IRWXO);
}

static int
ensure_service_logfile_exists(const char *proj_name, const char *serv_name)
{
    scoped char *logfile_path = get_service_logfile_path(proj_name, serv_name);
    int fd = open(logfile_path, O_APPEND | O_CREAT | O_WRONLY, S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH);
    close(fd);
    return fd > 0 ? 0 : 1;
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

    fprintf(fp, "%d\n%ld\n%ld", info.pid, info.c_time, info.s_time);
    fclose(fp);
    return 0;
}

#define MAX_META_LINE_LEN 1024 // TODO: this should be sufficient but probably should handle cases when it is not
static bool
try_parse_service_meta_file(const char *proj_name, const char *serv_name, struct service_process_info *info)
{
    info->pid = 0;
    info->c_time = 0;
    info->s_time = 0;

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

    if (fgets(buffer, MAX_META_LINE_LEN, fp))
    {
        buffer[strcspn(buffer, "\n")] = '\0';
        time_t s_time = atoll(buffer);
        if (s_time != 0)
        {
            info->s_time = s_time;
            parsed++;
        }
    }

    fclose(fp);
    // TODO: >= 2 is here to keep compatibility with older version.
    // Probably create some migration that will add 0 to new line of
    // each service meta file and changed next line to parsed === 3
    return parsed >= 2;
}

static FILE *
open_project_meta_file(const char *proj_name, const char *modes)
{
    scoped char *meta_file_path = get_project_meta_file_path(proj_name);
    return fopen(meta_file_path, modes);
}

static FILE *
open_service_meta_file(const char *proj_name, const char *serv_name, const char *modes)
{
    scoped char *meta_file_path = get_service_meta_file_path(proj_name, serv_name);
    return fopen(meta_file_path, modes);
}

static char *
get_project_dir_path(const char *proj_name)
{
    return str_printf("%s/%s", root_projects_dir, proj_name);
}

static char *
get_project_meta_file_path(const char *proj_name)
{
    return str_printf("%s/%s/%s", root_projects_dir, proj_name, meta_file_name);
}

static char *
get_service_dir_path(const char *proj_name, const char *serv_name)
{
    return str_printf("%s/%s/%s", root_projects_dir, proj_name, serv_name);
}

static char *
get_service_meta_file_path(const char *proj_name, const char *serv_name)
{
    return str_printf("%s/%s/%s/%s", root_projects_dir, proj_name, serv_name, meta_file_name);
}

static char *
get_service_logfile_path(const char *proj_name, const char *serv_name)
{
    return str_printf("%s/%s/%s/%s", root_projects_dir, proj_name, serv_name, logfile_name);
}

static bool
is_proccess_running(int pid)
{
    int status;
    pid_t res = waitpid(pid, &status, WNOHANG);
    return res == 0;
}

static int
remove_file_f(char *_path)
{
    scoped char *path = _path;
    return remove(path);
}

static int
remove_dir_f(char *_path)
{
    scoped char *path = _path;
    return rmdir(path);
}
