package micropub

import "io"

type fileWriter interface {
	// WriteFile is given the name and content-type of the uploaded media, along
	// with a Reader of its contents. It returns a URL locating the file, or an
	// error if a problem occured.
	WriteFile(name, contentType string, r io.Reader) (location string, err error)
}

type getDB interface {
	Entry(url string) (data map[string][]interface{}, err error)
}

type postDB interface {
	Create(data map[string][]interface{}) (string, error)
	Update(url string, replace, add, delete map[string][]interface{}, deleteAlls []string) error
	Delete(url string) error
	Undelete(url string) error
}
