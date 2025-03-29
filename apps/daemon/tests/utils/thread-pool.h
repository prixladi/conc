#ifndef THREAD_POOL_TESTS__H

char *test__thread_pool_create();

char *test__thread_pool_queue_job__idle_pool();
char *test__thread_pool_queue_job__idle_pool_over_capacity();

char *test__thread_pool_start__empty();
char *test__thread_pool_start__empty_running();

char *test__thread_pool_free__idle();
char *test__thread_pool_free__running();
char *test__thread_pool_free__idle_after_stop();

char *test__thread_pool__full_service1();
char *test__thread_pool__full_service2();
char *test__thread_pool__full_service3();
char *test__thread_pool__full_service4();
char *test__thread_pool__full_service5();
char *test__thread_pool__full_service6();
char *test__thread_pool__full_service7();
char *test__thread_pool__full_service8();

#endif
