package app

import (
	"github.com/stretchr/testify/assert"
	"testing"
)

func Test_FullPath(t *testing.T) {
	// Arrange
	gr := newTestGraph()

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
			ass := assert.New(t)

			// Act
			fp := gr.FullPath(test.node)

			// Assert
			ass.Equal(test.path, fp)
		})
	}
}

func Test_breadcrumbs(t *testing.T) {
	// Arrange
	ass := assert.New(t)
	gr := newTestGraph()

	var tests = []struct {
		path            string
		nodes           int
		expectedCurrent string
	}{
		{"/a/aa/", 2, "a"},
		{"/a/", 1, "a"},
		{"/b/", 1, "b"},
		{"/b/bb/", 2, "b"},
		{"", 1, ""},
		{"/", 1, ""},
	}

	for _, test := range tests {
		t.Run(test.path, func(t *testing.T) {
			// Act
			b, curr := breadcrumbs(gr, test.path)

			// Assert
			ass.Equal(test.nodes, len(b))
			ass.Equal(test.expectedCurrent, curr)
		})
	}
}

func newTestGraph() *Graph {
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
	return gr
}
