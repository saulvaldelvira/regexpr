#include "../target/include/bindings.h"
#include <stdio.h>

int main(void) {
        Regex *regex = regex_compile("abc.*");

        char* tests[] = {
                "abc",
                "abcc",
                "abcdds",
                NULL
        };

        for (char **ptr = tests; *ptr; ptr++) {
                printf("%s => %d\n", *ptr, regex_test(regex, *ptr));
        }

        regex_free(regex);
}
