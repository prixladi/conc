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

typedef struct ProcessDescriptor
{
    char *id;
    char *logfile_path;
    char **command;
    char *pwd;
} ProcessDescriptor;

static void handle_child(ProcessDescriptor pd);

static ProcessDescriptor proccess_descriptor_create(const char *project_name, const ServiceSettings settings, const char *logfile_path);

static void process_descriptor_free(ProcessDescriptor pd);

int process_start(const char *project_name, const ServiceSettings settings, const char *logfile_path)
{
    ProcessDescriptor pd = proccess_descriptor_create(project_name, settings, logfile_path);
    pid_t pid = fork();
    if (pid == 0)
    {
        handle_child(pd);
        LOG_ERROR("Unable to execute process '%s - %d', aborting", pd.id, pid);
        exit(127);
    }
    process_descriptor_free(pd);

    return pid;
}

static void handle_child(ProcessDescriptor pd)
{
    int current_pid = getpid();

    static const int OPEN_FLAGS = O_APPEND | O_CREAT | O_WRONLY;
    static const int CREATE_FLAGS = S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH;

    int fd = open(pd.logfile_path, OPEN_FLAGS, CREATE_FLAGS);
    if (fd <= 0)
    {
        LOG_ERROR("Unable to open log file '%s' for '%s - %d', aborting.\n", pd.logfile_path, pd.id, current_pid);
        process_descriptor_free(pd);
        exit(130);
    }

    LOG_DEBUG("Starting '%s - %d' with PID\n", pd.id, current_pid);

    dup2(fd, STDOUT_FILENO);
    dup2(fd, STDERR_FILENO);
    close(fd);

    if (pd.pwd)
        chdir(pd.pwd);

    execvp(pd.command[0], pd.command);
}

static const char *command_terminate = '\0';
static ProcessDescriptor proccess_descriptor_create(const char *project_name, const ServiceSettings settings, const char *logfile_path_i)
{
    char *id = str_concat(project_name, "/", settings.name, NULL);
    char *logfile_path = str_dup(logfile_path_i);
    char *pwd = str_dup(settings.pwd);

    size_t command_len = vector_length(settings.command);
    char **command = vector_create_prealloc(char *, command_len + 1);
    for (size_t i = 0; i < command_len; i++)
        vector_push_rval(command, str_dup(settings.command[i]));
    vector_push(command, command_terminate);

    ProcessDescriptor proc = {
        .id = id,
        .logfile_path = logfile_path,
        .command = command,
        .pwd = pwd};

    return proc;
}

static void process_descriptor_free(ProcessDescriptor pd)
{
    free(pd.id);
    free(pd.logfile_path);
    free(pd.pwd);

    vector_for_each(pd.command, free);
    vector_free(pd.command);

    pd.id = NULL;
    pd.logfile_path = NULL;
    pd.pwd = NULL;
    pd.command = NULL;
}
