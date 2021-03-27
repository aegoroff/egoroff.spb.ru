package domain

type Message struct {
	Type string
	Text string
}

type Apache struct {
	ID          string
	Stylesheet  string
	Title       string
	Description string
	Keywords    string
}

type SiteSection struct {
	Key      int64
	Id       string         `json:"id"`
	Class    string         `json:"class"`
	Title    string         `json:"title"`
	Descr    string         `json:"descr"`
	Keywords string         `json:"keywords"`
	Children []*SiteSection `json:"children"`
}

// ID gets section's key
func (s *SiteSection) ID() int64 {
	return s.Key
}
