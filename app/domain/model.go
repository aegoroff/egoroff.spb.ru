package domain

import (
	"encoding/xml"
)

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

type AuthenticatedUserInfo struct {
	Id        int64  `json:"id"`
	Created   string `json:"created"`
	Admin     bool   `json:"admin"`
	AvatarUrl string `json:"avatarUrl"`
	Email     string `json:"email"`
	Name      string `json:"name"`
	Username  string `json:"username"`
	Verified  bool   `json:"verified"`
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

type Feed struct {
	XMLName xml.Name    `xml:"feed"`
	XmlNS   string      `xml:"xmlns,attr"`
	Title   FeedTitle   `xml:"title"`
	Id      string      `xml:"id"`
	Updated string      `xml:"updated"`
	Link    []FeedLink  `xml:"link"`
	Entries []FeedEntry `xml:"entry"`
}

type FeedEntry struct {
	XMLName   xml.Name    `xml:"entry"`
	XmlBase   string      `xml:"xml:base,attr"`
	Title     FeedTitle   `xml:"title"`
	Id        string      `xml:"id"`
	Updated   string      `xml:"updated"`
	Published string      `xml:"published"`
	Content   FeedContent `xml:"content"`
	Author    FeedAuthor  `xml:"author"`
}

type FeedLink struct {
	XMLName xml.Name `xml:"link"`
	Href    string   `xml:"href,attr"`
	Rel     string   `xml:"rel,attr,omitempty"`
}

type FeedTitle struct {
	Title   string   `xml:",chardata"`
	XMLName xml.Name `xml:"title"`
	Type    string   `xml:"type,attr"`
}

type FeedContent struct {
	Content string   `xml:",chardata"`
	XMLName xml.Name `xml:"content"`
	Type    string   `xml:"type,attr"`
}

type FeedAuthor struct {
	XMLName xml.Name `xml:"author"`
	Name    string   `xml:"name"`
}
