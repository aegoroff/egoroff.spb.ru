package db

import (
	"egoroff.spb.ru/app/domain"
	"github.com/vcraescu/go-paginator/v2"
	"log"
)

type Poster struct {
	pager paginator.Paginator
}

func NewPoster(page int) *Poster {
	return &Poster{
		pager: paginator.New(NewPostsAdaptor(), page),
	}
}

func NewCustomPoster(adaptor *DatastoreAdaptor, page int) *Poster {
	return &Poster{
		pager: paginator.New(adaptor, page),
	}
}

func (p *Poster) HasPages() bool {
	r, _ := p.pager.HasPages()
	return r
}

func (p *Poster) HasPrev() bool {
	r, _ := p.pager.HasPrev()
	return r
}

func (p *Poster) HasNext() bool {
	r, _ := p.pager.HasNext()
	return r
}

func (p *Poster) PrevPage() int {
	r, _ := p.pager.PrevPage()
	return r
}

func (p *Poster) NextPage() int {
	r, _ := p.pager.NextPage()
	return r
}

func (p *Poster) Page() int {
	r, _ := p.pager.Page()
	return r
}

func (p *Poster) SetPage(page int) {
	p.pager.SetPage(page)
}

func (p *Poster) PageNums() int {
	r, _ := p.pager.PageNums()
	return r
}

func (p *Poster) Pages() []int {
	return makeRange(1, p.PageNums())
}

func (p *Poster) Posts() []*domain.SmallPost {
	var posts []*domain.SmallPost
	err := p.pager.Results(&posts)
	if err != nil {
		log.Println(err)
	}
	return posts
}

func makeRange(min, max int) []int {
	a := make([]int, max-min+1)
	for i := range a {
		a[i] = min + i
	}
	return a
}
