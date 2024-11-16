#include <stdlib.h>
#include <unistd.h>
#include <stdbool.h>
#include <string.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <fcntl.h>
#include <sys/stat.h>

#include "utils/log.h"
#include "utils/fs.h"
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

static struct process_descriptor pd_create(const struct project_settings project, const struct service_settings service,
                                           const struct env_variable *env, const char *logfile);
static char **env_pair_create(struct env_variable var);

static void pd_free(struct process_descriptor pd);

int
process_start(const struct project_settings project, const struct service_settings settings,
              const struct env_variable *env, const char *logfile_path)
{
    struct process_descriptor pd = pd_create(project, settings, env, logfile_path);
    pid_t pid = fork();
    if (pid == 0)
    {
        handle_child(pd);
        log_critical("Unable to execute process '%s' with pid %d and pwd '%s', aborting", pd.id, pid, pd.pwd);
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
pd_create(const struct project_settings project, const struct service_settings service,
          const struct env_variable *c_env, const char *logfile_path_i)
{
    char **command = vec_create(char *);
    for (size_t i = 0; i < vec_length(service.command); i++)
        vec_push(command, str_dup(service.command[i]));
    vec_push(command, command_terminate);

    char ***env = vec_create(char **);
    for (size_t i = 0; i < vec_length(service.env); i++)
    {
        char **env_pair = env_pair_create(service.env[i]);
        vec_push(env, env_pair);
    }
    for (size_t i = 0; i < vec_length(project.env); i++)
    {
        for (size_t j = 0; j < vec_length(env); j++)
            if (strcmp(env[j][0], project.env[i].key) == 0)
                continue;
        char **env_pair = env_pair_create(project.env[i]);
        vec_push(env, env_pair);
    }
    for (size_t i = 0; i < vec_length(c_env); i++)
    {
        for (size_t j = 0; j < vec_length(env); j++)
            if (strcmp(env[j][0], c_env[i].key) == 0)
                continue;
        char **env_pair = env_pair_create(c_env[i]);
        vec_push(env, env_pair);
    }

    char *pwd = service.pwd && is_path_absolute(service.pwd) ? str_dup(service.pwd)
                                                             : paths_join(project.cwd, service.pwd);

    struct process_descriptor proc = {
        .id = str_printf("%s/%s", project.name, service.name),
        .logfile_path = str_dup(logfile_path_i),
        .command = command,
        .env = env,
        .pwd = pwd,
    };

    return proc;
}

static char **
env_pair_create(struct env_variable env)
{
    char **env_pair = malloc(sizeof(char *) * 2);
    env_pair[0] = str_dup(env.key);
    env_pair[1] = str_dup(env.value);
    return env_pair;
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
