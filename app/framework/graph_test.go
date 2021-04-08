package framework

import (
	"egoroff.spb.ru/app/domain"
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
			b, curr := Breadcrumbs(gr, test.path)

			// Assert
			ass.Equal(test.nodes, len(b))
			ass.Equal(test.expectedCurrent, curr)
		})
	}
}

func newTestGraph() *Graph {
	root := domain.SiteSection{
		Id:       "/",
		Children: make([]*domain.SiteSection, 0),
	}
	a := domain.SiteSection{
		Id:       "a",
		Children: make([]*domain.SiteSection, 0),
	}
	root.Children = append(root.Children, &a)
	b := domain.SiteSection{
		Id:       "b",
		Children: make([]*domain.SiteSection, 0),
	}
	root.Children = append(root.Children, &b)
	aa := domain.SiteSection{
		Id: "aa",
	}
	a.Children = append(a.Children, &aa)
	bb := domain.SiteSection{
		Id: "bb",
	}
	b.Children = append(b.Children, &bb)

	gr := NewGraph(&root)
	return gr
}
