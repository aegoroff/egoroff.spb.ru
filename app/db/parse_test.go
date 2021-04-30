package db

import (
	"github.com/stretchr/testify/assert"
	"testing"
)

func Test_ExtractPostID_success(t *testing.T) {
	var tests = []struct {
		uri string
		id  int64
	}{
		{"/100.html", 100},
		{"100.html", 100},
		{"/blog/100.html", 100},
		{"blog/100.html", 100},
		{"https://www.egoroff.spb.ru/blog/100.html", 100},
		{"https://www.egoroff.spb.ru/blog/5866158766948352.html", 5866158766948352},
		{"https://www.egoroff.spb.ru/100.html", 100},
		{"https://www.egoroff.spb.ru/5866158766948352.html", 5866158766948352},
		{"https://www.egoroff.spb.ru/5866158766948352", 5866158766948352},
	}

	for _, test := range tests {
		t.Run(test.uri, func(t *testing.T) {
			// Arrange
			ass := assert.New(t)

			// Act
			id, err := ExtractPostID(test.uri)

			// Assert
			ass.Equal(test.id, id)
			ass.NoError(err)
		})
	}
}

func Test_ExtractPostID_failure(t *testing.T) {
	var tests = []struct {
		uri string
	}{
		{"/q.html"},
		{"/10q.html"},
		{"/q10.html"},
		{"https://www.egoroff.spb.ru/q10.html"},
		{"https://www.egoroff.spb.ru/blog/q10.html"},
		{"https:www.egoroff.spb.ru/blog/10.html"},
		{"+not_uri"},
	}

	for _, test := range tests {
		t.Run(test.uri, func(t *testing.T) {
			// Arrange
			ass := assert.New(t)

			// Act
			id, err := ExtractPostID(test.uri)

			// Assert
			ass.Equal(int64(0), id)
			ass.Error(err)
		})
	}
}
