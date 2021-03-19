package app

import (
	"gonum.org/v1/gonum/graph/path"
	"gonum.org/v1/gonum/graph/simple"
)

// Graph provides site sections graph
type Graph struct {
	g      *simple.DirectedGraph
	search map[string]int64
	nextID int64
}

// New creates new graph
func NewGraph(root *SiteSection) *Graph {
	gr := &Graph{
		g:      simple.NewDirectedGraph(),
		nextID: 1,
		search: make(map[string]int64),
	}

	gr.newNode(root)

	gr.newEdges(root)

	return gr
}

// AllPaths returns a shortest-path tree for shortest paths in the graph g.
// If the graph does not implement graph.Weighter, UniformCost is used.
// AllPaths will panic if g has a negative edge weight.
//
// The time complexity of AllPaths is O(|V|.|E|+|V|^2.log|V|).
func (gr *Graph) AllPaths() *path.AllShortest {
	paths := path.DijkstraAllPaths(gr.g)
	return &paths
}

// To returns all nodes in g that can reach directly to n.
func (gr *Graph) To(n *SiteSection) []*SiteSection {
	refs := gr.g.To(n.ID())
	nodes := make([]*SiteSection, refs.Len())
	i := 0
	for refs.Next() {
		nodes[i] = refs.Node().(*SiteSection)
		i++
	}
	return nodes
}

func (gr *Graph) Section(id string) *SiteSection {
	key, ok := gr.search[id]
	if !ok {
		return nil
	}
	return gr.g.Node(key).(*SiteSection)
}

func (gr *Graph) newNode(s *SiteSection) {
	s.key = gr.nextID
	gr.search[s.Id] = gr.nextID
	gr.nextID++
	gr.g.AddNode(s)
}

func (gr *Graph) newEdges(root *SiteSection) {
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
