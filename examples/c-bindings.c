#include "../target/include/bindings.h"
#include <stdio.h>

int main(void) {
        Regex *regex = regex_compile("(abc|def)");

        char* tests[] = {
                "abc",
                "abcc",
                "abcabc",
                "abcdefabc",
                "abcdds",
                NULL
        };

        printf("Regular expression: (abc|def)\n");

        for (char **ptr = tests; *ptr; ptr++) {
                bool matches = regex_test(regex, *ptr);
                if (!matches)
                        continue;
                char *test = *ptr;
                Span span;
                RegexMatcher *matcher = regex_find_matches(regex, test);
                printf("Matches of %s\n", test);
                while (regex_matcher_next(matcher, &span)) {
                        printf("[%d:%d] %.*s\n",
                                (int)span.offset,
                                (int)(span.offset + span.len),
                                (int)span.len,
                                &test[span.offset]
                        );
                }
                regex_matcher_free(matcher);
        }
        regex_free(regex);
}
