package micropub

import (
	"net/url"
	"path/filepath"
	"strconv"
)

func extractPostID(u string) (int64, error) {
	uri, err := url.Parse(u)
	if err != nil {
		return 0, err
	}
	base := filepath.Base(uri.Path)
	ext := filepath.Ext(base)

	id, err := strconv.ParseInt(base[:len(base)-len(ext)], 10, 64)
	if err != nil {
		return 0, err
	}
	return id, nil
}
