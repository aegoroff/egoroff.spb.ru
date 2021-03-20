package app

import (
	"cloud.google.com/go/datastore"
	"github.com/vcraescu/go-paginator/v2"
	"log"
	"time"
)

type Repository struct {
	conn *Connection
}

func NewRepository() *Repository {
	return &Repository{
		conn: NewConnection(5 * time.Second),
	}
}

func (r *Repository) Config() (*Config, error) {
	c, err := r.conn.Connect()
	if err != nil {
		return nil, err
	}
	defer Close(c)
	k := datastore.NameKey("Config", "master", nil)
	var config Config

	ctx, cancel := r.conn.newContext()
	defer cancel()

	err = c.Get(ctx, k, &config)
	if err != nil {
		return nil, err
	}

	return &config, nil
}

func (r *Repository) Tags() []*TagContainter {
	c, err := r.conn.Connect()
	if err != nil {
		log.Println(err)
		return nil
	}
	defer Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	q := datastore.NewQuery("Post").Filter("is_public=", true).Project("tags", "created", "__key__")
	var containters []*TagContainter
	_, err = c.GetAll(ctx, q, &containters)
	if err != nil {
		log.Println(err)
	}
	return containters
}

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

func makeRange(min, max int) []int {
	a := make([]int, max-min+1)
	for i := range a {
		a[i] = min + i
	}
	return a
}

func (p *Poster) Posts() []*SmallPost {
	var posts []*SmallPost
	err := p.pager.Results(&posts)
	if err != nil {
		log.Println(err)
	}
	return posts
}

type DatastoreAdaptor struct {
	conn  *Connection
	query *datastore.Query
}

func NewDatastoreAdaptor(query *datastore.Query) *DatastoreAdaptor {
	return &DatastoreAdaptor{
		conn:  NewConnection(5 * time.Second),
		query: query,
	}
}

func NewPostsAdaptor() *DatastoreAdaptor {
	q := datastore.NewQuery("Post").Filter("is_public=", true).Order("-created")
	return NewDatastoreAdaptor(q)
}

func (a *DatastoreAdaptor) Nums() (int64, error) {
	c, err := a.conn.Connect()
	if err != nil {
		return 0, err
	}
	defer Close(c)

	ctx, cancel := a.conn.newContext()
	defer cancel()

	count, err := c.Count(ctx, a.query)
	return int64(count), err
}

func (a *DatastoreAdaptor) Slice(offset, length int, data interface{}) error {
	c, err := a.conn.Connect()
	if err != nil {
		return err
	}
	defer Close(c)

	ctx, cancel := a.conn.newContext()
	defer cancel()

	_, err = c.GetAll(ctx, a.query.Limit(length).Offset(offset), data)
	return err
}
