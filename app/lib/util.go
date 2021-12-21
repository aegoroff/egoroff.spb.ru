package lib

import (
	"io"
	"log"
	"strings"
)

// Close wraps io.Closer Close func with error handling
func Close(c io.Closer) {
	if c == nil {
		return
	}
	err := c.Close()
	if err != nil {
		log.Println(err)
	}
}

// RemoveLineBreaks removes line breaks from string to be saved into text log
func RemoveLineBreaks(s string) string {
	replace := strings.Replace(s, "\n", "", -1)
	replace = strings.Replace(replace, "\r", "", -1)
	return replace
}
