#ifndef PROCESS__H
#define PROCESS__H

#include "settings.h"

int process_start(const char *project_name, const ServiceSettings settings, const char *logfile_path);

#endif