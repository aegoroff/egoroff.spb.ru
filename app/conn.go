package app

import (
	"cloud.google.com/go/datastore"
	"context"
	"time"
)

// Sets your Google Cloud Platform project ID.
const projectID = "egoroff"

type Connection struct {
	timeout time.Duration
}

func NewConnection(timeout time.Duration) *Connection {
	return &Connection{timeout: timeout}
}

func (c *Connection) Connect() (*datastore.Client, error) {
	ctx, cancel := c.newContext()
	defer cancel()

	// Create a datastore client. In a typical application, you would create
	// a single client which is reused for every datastore operation.
	return datastore.NewClient(ctx, projectID)
}

func (c *Connection) newContext() (context.Context, context.CancelFunc) {
	ctx, cancel := context.WithTimeout(context.Background(), c.timeout)
	return ctx, cancel
}
