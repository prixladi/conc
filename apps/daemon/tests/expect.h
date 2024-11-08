#ifndef EXPECT__H

#define expect(pred, message) \
    do \
    { \
        __auto_type res = pred; \
        if (!res) \
            return message; \
    } while (0)

#endif
