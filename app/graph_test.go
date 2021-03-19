package app

import (
	"github.com/stretchr/testify/assert"
	"testing"
)

func Test_FullPath(t *testing.T) {
	// Arrange
	ass := assert.New(t)
	root := SiteSection{
		Id:       "/",
		Children: make([]*SiteSection, 0),
	}
	a := SiteSection{
		Id:       "a",
		Children: make([]*SiteSection, 0),
	}
	root.Children = append(root.Children, &a)
	b := SiteSection{
		Id:       "b",
		Children: make([]*SiteSection, 0),
	}
	root.Children = append(root.Children, &b)
	aa := SiteSection{
		Id: "aa",
	}
	a.Children = append(a.Children, &aa)
	bb := SiteSection{
		Id: "bb",
	}
	b.Children = append(b.Children, &bb)

	gr := NewGraph(&root)

	var tests = []struct {
		node string
		path string
	}{
		{"aa", "/a/aa/"},
		{"a", "/a/"},
		{"b", "/b/"},
		{"bb", "/b/bb/"},
		{"ab", ""},
		{"/", "/"},
	}

	for _, test := range tests {
		t.Run(test.node, func(t *testing.T) {
			// Act
			fp := gr.FullPath(test.node)

			// Assert
			ass.Equal(test.path, fp)
		})
	}
}
