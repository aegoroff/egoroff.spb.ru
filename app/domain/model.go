package domain

import "encoding/xml"

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

type AuthenticatedUser struct {
	LoginOrName   string `json:"loginOrName"`
	Authenticated bool   `json:"authenticated"`
}

type Navigation struct {
	Sections    []*SiteSection `json:"sections"`
	Breadcrumbs []*SiteSection `json:"breadcrumbs"`
}

type SiteSection struct {
	Key      int64
	Id       string         `json:"id"`
	Icon     string         `json:"icon"`
	Title    string         `json:"title"`
	Descr    string         `json:"descr"`
	Keywords string         `json:"keywords"`
	Active   bool           `json:"active"`
	Children []*SiteSection `json:"children"`
}

// ID gets section's key
func (s *SiteSection) ID() int64 {
	return s.Key
}

type UrlSet struct {
	XMLName xml.Name `xml:"urlset"`
	XmlNS   string   `xml:"xmlns,attr"`
	Urls    []Url    `xml:"url"`
}

type Url struct {
	XMLName    xml.Name `xml:"url"`
	Location   string   `xml:"loc"`
	ChangeFreq string   `xml:"changefreq"`
	Priority   float64  `xml:"priority"`
}
