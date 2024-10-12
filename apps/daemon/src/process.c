#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <fcntl.h>
#include <sys/stat.h>

#include "utils/log.h"
#include "utils/string.h"
#include "utils/vector.h"

#include "process.h"

static const char *command_terminate = NULL;

struct process_descriptor
{
    char *id;
    char *logfile_path;
    char ***env;
    char **command;
    char *pwd;
};

static void handle_child(struct process_descriptor pd);

static struct process_descriptor pd_create(const char *proj_name, const struct service_settings settings,
                                           const char *logfile);

static void pd_free(struct process_descriptor pd);

int
process_start(const char *proj_name, const struct service_settings settings, const char *logfile_path)
{
    struct process_descriptor pd = pd_create(proj_name, settings, logfile_path);
    pid_t pid = fork();
    if (pid == 0)
    {
        handle_child(pd);
        log_critical("Unable to execute process '%s - %d', aborting", pd.id, pid);
        pd_free(pd);
        exit(127);
    }

    pd_free(pd);
    return pid;
}

static void
handle_child(struct process_descriptor pd)
{
    int current_pid = getpid();

    static const int OPEN_FLAGS = O_APPEND | O_CREAT | O_WRONLY;
    static const int CREATE_FLAGS = S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH;

    int fd = open(pd.logfile_path, OPEN_FLAGS, CREATE_FLAGS);
    if (fd <= 0)
    {
        log_error("Unable to open log file '%s' for '%s - %d', aborting.\n", pd.logfile_path, pd.id, current_pid);
        return;
    }

    log_debug("Starting process '%s - %d'\n", pd.id, current_pid);

    for (size_t i = 0; i < vec_length(pd.env); i++)
    {
        char **pair = pd.env[i];
        setenv(pair[0], pair[1], 1);
    }

    dup2(fd, STDOUT_FILENO);
    dup2(fd, STDERR_FILENO);
    close(fd);

    if (pd.pwd)
        chdir(pd.pwd);

    execvp(pd.command[0], pd.command);
}

static struct process_descriptor
pd_create(const char *proj_name, const struct service_settings settings, const char *logfile_path_i)
{
    char *id = str_printf("%s/%s", proj_name, settings.name);
    char *logfile_path = str_dup(logfile_path_i);
    char *pwd = str_dup(settings.pwd);

    size_t command_len = vec_length(settings.command);
    char **command = vec_create_prealloc(char *, command_len + 1);
    for (size_t i = 0; i < command_len; i++)
        vec_push_rval(command, str_dup(settings.command[i]));
    vec_push(command, command_terminate);

    size_t env_len = vec_length(settings.env);
    char ***env = vec_create_prealloc(char **, env_len);
    for (size_t i = 0; i < env_len; i++)
    {
        char **env_pair = malloc(sizeof(char *) * 2);
        env_pair[0] = str_dup(settings.env[i].key);
        env_pair[1] = str_dup(settings.env[i].value);
        vec_push(env, env_pair);
    }
    vec_push(command, command_terminate);

    struct process_descriptor proc = {
        .id = id,
        .logfile_path = logfile_path,
        .command = command,
        .pwd = pwd,
        .env = env,
    };

    return proc;
}

static void
pd_free(struct process_descriptor pd)
{
    free(pd.id);
    free(pd.logfile_path);
    free(pd.pwd);

    for (size_t i = 0; i < vec_length(pd.env); i++)
    {
        char **pair = pd.env[i];
        free(pair[0]);
        free(pair[1]);
        free(pair);
    }
    vec_free(pd.env);

    vec_for_each(pd.command, free);
    vec_free(pd.command);

    pd.id = NULL;
    pd.logfile_path = NULL;
    pd.pwd = NULL;
    pd.command = NULL;
}