package app

import (
	"cloud.google.com/go/datastore"
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

func (r *Repository) Posts(limit int) ([]*Post, error) {
	c, err := r.conn.Connect()
	if err != nil {
		return nil, err
	}
	defer Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	var posts []*Post
	_, err = c.GetAll(ctx, datastore.NewQuery("Post").Order("-created").Limit(limit), &posts)
	if err != nil {
		return nil, err
	}

	return posts, nil
}
