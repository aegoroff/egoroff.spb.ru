package db

import (
	"io"
)

type FileWriter struct {
}

func NewFileWriter() *FileWriter {
	return &FileWriter{}
}

func (f *FileWriter) WriteFile(name, contentType string, r io.Reader) (location string, err error) {
	return "https://www.egoroff.spb.ru", nil
}
