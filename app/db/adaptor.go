package db

import (
	"cloud.google.com/go/datastore"
	"egoroff.spb.ru/app/lib"
	"time"
)

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
