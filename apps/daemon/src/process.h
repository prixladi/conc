#ifndef PROCESS__H
#define PROCESS__H

#include "settings.h"

int process_start(const struct project_settings project, const struct service_settings settings,
                  const struct env_variable *env, const char *logfile_path);

#endif
