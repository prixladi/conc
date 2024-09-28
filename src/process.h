#ifndef PROCESS__H
#define PROCESS__H

#include "settings.h"

int process_start(const char *proj_name, const struct service_settings settings, const char *logfile_path);

#endif
