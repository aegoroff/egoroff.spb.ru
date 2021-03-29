package db

import (
	"cloud.google.com/go/datastore"
	"egoroff.spb.ru/app/domain"
	"egoroff.spb.ru/app/lib"
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

func (r *Repository) Config() (*domain.Config, error) {
	c, err := r.conn.Connect()
	if err != nil {
		return nil, err
	}
	defer lib.Close(c)
	k := datastore.NameKey("Config", "master", nil)
	var config domain.Config

	ctx, cancel := r.conn.newContext()
	defer cancel()

	err = c.Get(ctx, k, &config)
	if err != nil {
		return nil, err
	}

	return &config, nil
}

func (r *Repository) UserByFederatedId(sub string) (*domain.User, error) {
	c, err := r.conn.Connect()
	if err != nil {
		return nil, err
	}
	defer lib.Close(c)
	var users []*domain.User

	ctx, cancel := r.conn.newContext()
	defer cancel()

	q := datastore.NewQuery("User").Filter("federated_id=", sub)
	_, err = c.GetAll(ctx, q, &users)
	if err != nil || len(users) == 0 {
		return nil, err
	}

	return users[0], nil
}

func (r *Repository) NewUser(user *domain.User) error {
	c, err := r.conn.Connect()
	if err != nil {
		return err
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	k := datastore.IncompleteKey("User", nil)
	_, err = c.Put(ctx, k, user)
	if err != nil {
		return nil
	}

	return nil
}

func (r *Repository) Tags() []*domain.TagContainter {
	c, err := r.conn.Connect()
	if err != nil {
		log.Println(err)
		return nil
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	q := datastore.NewQuery("Post").Filter("is_public=", true).Project("tags", "created", "__key__")
	var containters []*domain.TagContainter
	_, err = c.GetAll(ctx, q, &containters)
	if err != nil {
		log.Println(err)
	}
	return containters
}

func (r *Repository) PostKeys() []*datastore.Key {
	c, err := r.conn.Connect()
	if err != nil {
		log.Println(err)
		return nil
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	q := datastore.NewQuery("Post").KeysOnly()
	var posts []*domain.Post
	keys, err := c.GetAll(ctx, q, &posts)
	if err != nil {
		log.Println(err)
	}
	return keys
}

func (r *Repository) Folders() []*domain.Folder {
	c, err := r.conn.Connect()
	if err != nil {
		log.Println(err)
		return nil
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	q := datastore.NewQuery("Folder").Filter("is_public=", true)
	var folders []*domain.Folder
	_, err = c.GetAll(ctx, q, &folders)
	if err != nil {
		log.Println(err)
	}
	return folders
}

func (r *Repository) File(k *datastore.Key) *domain.File {
	c, err := r.conn.Connect()
	if err != nil {
		log.Println(err)
		return nil
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	var file domain.File
	err = c.Get(ctx, k, &file)
	if err != nil {
		log.Println(err)
	}
	return &file
}

func (r *Repository) Post(id int64) *domain.Post {
	c, err := r.conn.Connect()
	if err != nil {
		log.Println(err)
		return nil
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	k := datastore.IDKey("Post", id, nil)
	var post domain.Post
	err = c.Get(ctx, k, &post)
	if err != nil {
		log.Println(err)
	}
	return &post
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

func (p *Poster) Posts() []*domain.SmallPost {
	var posts []*domain.SmallPost
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
	defer lib.Close(c)

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
	defer lib.Close(c)

	ctx, cancel := a.conn.newContext()
	defer cancel()

	_, err = c.GetAll(ctx, a.query.Limit(length).Offset(offset), data)
	return err
}
