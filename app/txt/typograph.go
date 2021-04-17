package txt

import (
	"fmt"
	"regexp"
)

var skips = map[string]struct{}{
	"pre":    {},
	"a":      {},
	"script": {},
	"code":   {},
	"style":  {},
}

func skipElement(elt string) bool {
	_, ok := skips[elt]
	return ok
}

var replaces = map[*regexp.Regexp]string{
	regexp.MustCompile("(\\w)-(\\s+)"):                "$1 -$2",
	regexp.MustCompile("\\+-"):                        entity("plusmn"),
	regexp.MustCompile("(\\s+)(--?|—|-)(\\s|\u00a0)"): entity("nbsp") + entity("mdash") + "$3",
	regexp.MustCompile("(^)(--?|—|-)(\\s|\u00a0)"):    entity("mdash") + "$3",
	regexp.MustCompile("\\.{2,}"):                     entity("hellip"),
	regexp.MustCompile("(\\d)-(\\d)"):                 "$1" + entity("minus") + "$2",
}

func typo(s string) string {
	result := s
	for reg, repl := range replaces {
		result = reg.ReplaceAllString(result, repl)
	}

	return result
}

var sym = map[string]string{
	"1/2": "frac12",
	"1/4": "frac14",
	"3/4": "frac34",
}

func entity(s string) string {
	symbol := s
	if r, ok := sym[s]; ok {
		symbol = r
	}
	return fmt.Sprintf("&%s;", symbol)
}
