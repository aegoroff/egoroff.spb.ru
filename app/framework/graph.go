package framework

import (
	"egoroff.spb.ru/app/domain"
	"gonum.org/v1/gonum/graph/path"
	"gonum.org/v1/gonum/graph/simple"
	"strings"
)

// Graph provides site sections graph
type Graph struct {
	g      *simple.DirectedGraph
	search map[string]int64
	nextID int64
}

// New creates new graph
func NewGraph(root *domain.SiteSection) *Graph {
	gr := &Graph{
		g:      simple.NewDirectedGraph(),
		nextID: 1,
		search: make(map[string]int64),
	}

	gr.newNode(root)

	gr.newEdges(root)

	return gr
}

func (gr *Graph) Section(id string) *domain.SiteSection {
	key, ok := gr.search[id]
	if !ok {
		return nil
	}
	return gr.g.Node(key).(*domain.SiteSection)
}

func (gr *Graph) FullPath(id string) string {
	s := gr.Section(id)
	if s == nil {
		return ""
	}

	gpaths := path.DijkstraFrom(gr.g.Node(1), gr.g)
	var paths []string
	nodes, _ := gpaths.To(s.ID())
	for _, n := range nodes {
		if n.ID() == 1 {
			continue
		}
		paths = append(paths, n.(*domain.SiteSection).Id)
	}

	sep := "/"
	if len(paths) == 0 {
		return sep
	}
	return sep + strings.Join(paths, sep) + sep
}

func (gr *Graph) newNode(s *domain.SiteSection) {
	s.Key = gr.nextID
	gr.search[s.Id] = gr.nextID
	gr.nextID++
	gr.g.AddNode(s)
}

func (gr *Graph) newEdges(root *domain.SiteSection) {
	if root.Children == nil {
		return
	}

	for _, child := range root.Children {
		gr.newNode(child)
		gr.newEdges(child)

		e := gr.g.NewEdge(root, child)
		gr.g.SetEdge(e)
	}
}
