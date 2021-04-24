package app

import (
	"cloud.google.com/go/datastore"
	"egoroff.spb.ru/app/auth"
	"egoroff.spb.ru/app/db"
	"egoroff.spb.ru/app/domain"
	"egoroff.spb.ru/app/framework"
	"github.com/gin-gonic/gin"
	"github.com/patrickmn/go-cache"
	"log"
	"net/http"
	"sort"
	"strconv"
	"time"
)

var tagRanks = []string{"tagRank10", "tagRank9",
	"tagRank8", "tagRank7", "tagRank6",
	"tagRank5", "tagRank4", "tagRank3",
	"tagRank2", "tagRank1"}

var months = []string{"Январь", "Февраль",
	"Март", "Апрель", "Май",
	"Июнь", "Июль", "Август",
	"Сентябрь", "Октябрь", "Ноябрь", "Декабрь"}

type api struct {
	cache *cache.Cache
}

func NewApi() Router {
	c := cache.New(10*time.Minute, 10*time.Minute)
	return &api{cache: c}
}

func (a *api) Route(r *gin.Engine) {
	ap := r.Group("/api/v2")
	{
		ap.GET("/", a.index)
		ap.GET("/navigation/", a.navigation)

		b := ap.Group("/blog")
		{
			b.GET("/posts/", a.posts)
			b.GET("/archive/", a.archive)
		}
		authGrp := ap.Group("/auth")
		{
			authGrp.GET("/user", a.user)
			authGrp.GET("/userinfo", a.userInfo)
			authGrp.PUT("/userinfo", a.userInfoUpdate)
		}
	}
}
func (a *api) index(c *gin.Context) {
	ctx := framework.NewContext(c)

	ctx["html_class"] = "welcome"
	ctx["request"] = c.Request
	ctx["title"] = "API"

	c.HTML(http.StatusOK, "api/index.html", ctx)
}

func (*api) user(c *gin.Context) {
	result := domain.AuthorizedUser{}
	auth.IfAuthorized(c, func(user *domain.User) {
		result = domain.AuthorizedUser{
			LoginOrName:   user.String(),
			Authenticated: true,
			Provider:      user.Provider,
			Admin:         user.Admin,
		}
	})

	c.JSON(http.StatusOK, result)
}

func (*api) userInfo(c *gin.Context) {
	result := domain.AuthorizedUserInfo{}
	auth.IfAuthorized(c, func(user *domain.User) {
		result = domain.AuthorizedUserInfo{
			Id:        user.Key.ID,
			Created:   user.Created.Format(time.RFC3339),
			Admin:     user.Admin,
			AvatarUrl: user.AvatarUrl,
			Email:     user.Email,
			Name:      user.Name,
			Username:  user.Username,
			Verified:  user.Verified,
			Provider:  user.Provider,
		}
	})

	c.JSON(http.StatusOK, result)
}

func (*api) userInfoUpdate(c *gin.Context) {
	auth.IfAuthorized(c, func(user *domain.User) {
		var req domain.AuthorizedUserInfo
		err := c.Bind(&req)
		if err != nil {
			log.Println(err)
		} else {
			user.Name = req.Name
			user.Email = req.Email
			user.AvatarUrl = req.AvatarUrl
			repo := db.NewRepository()
			err = repo.UpdateUser(user, user.Key)
			if err != nil {
				log.Println(err)
			}
		}
	})
}

func (a *api) navigation(c *gin.Context) {
	uri := c.Query("uri")

	siteMap := framework.ReadSiteMap()
	gr := framework.NewGraph(siteMap)
	bc, curr := framework.Breadcrumbs(gr, uri)

	nav := domain.Navigation{
		Sections: siteMap.Children,
	}

	if uri != "/" {
		nav.Breadcrumbs = bc
	}

	for _, section := range nav.Sections {
		if section.Id == curr {
			section.Active = true
			break
		}
	}

	c.JSON(http.StatusOK, nav)
}

func (a *api) posts(c *gin.Context) {
	page, err := strconv.ParseInt(c.Query("page"), 10, 32)
	if err != nil {
		page = 1
	}
	limit, err := strconv.ParseInt(c.Query("limit"), 10, 32)
	if err != nil {
		limit = 20
	}
	offset, err := strconv.ParseInt(c.Query("offset"), 10, 32)

	q := datastore.NewQuery("Post").Filter("is_public=", true).Order("-created")
	if offset > 0 {
		q = q.Offset(int(offset))
	}

	y, _ := strconv.ParseInt(c.Query("year"), 10, 32)
	m, _ := strconv.ParseInt(c.Query("month"), 10, 32)

	if y != 0 {
		loc := time.Now().Location()
		var currMonth time.Time
		var nextMonth time.Time
		if m != 0 {
			currMonth = time.Date(int(y), time.Month(m), 1, 0, 0, 0, 0, loc)
			nextMonth = currMonth.AddDate(0, 1, 0)
		} else {
			currMonth = time.Date(int(y), time.January, 1, 0, 0, 0, 0, loc)
			nextMonth = time.Date(int(y+1), time.January, 1, 0, 0, 0, 0, loc)
		}
		q = q.Filter("created>=", currMonth).Filter("created<", nextMonth)
	} else {
		tag := c.Query("tag")
		if tag != "" {
			q = q.Filter("tags=", tag)
		}
	}

	adaptor := db.NewDatastoreAdaptor(q)
	poster := db.NewCustomPoster(adaptor, int(limit))
	poster.SetPage(int(page))

	posts := poster.Posts()
	for _, post := range posts {
		post.Id = post.Key.ID
		post.ShortText = post.Html()
	}

	c.JSON(http.StatusOK, ApiResult{
		Status: "success",
		Count:  len(posts),
		Page:   int(page),
		Pages:  poster.PageNums(),
		Now:    time.Now(),
		Result: posts,
	})
}

func (a *api) archive(c *gin.Context) {
	k := "archive"
	arch, ok := a.cache.Get(k)
	if ok {
		c.JSON(http.StatusOK, arch)
		return
	}
	rep := db.NewRepository()
	containers := rep.Tags()
	tags, times, total := groupContainers(containers)
	t := createTags(tags, total)
	y := createArchive(times)
	arch = Archive{
		Tags:  t,
		Years: y,
	}
	a.cache.Set(k, arch, cache.DefaultExpiration)
	c.JSON(http.StatusOK, arch)
}

type ApiResult struct {
	Status string      `json:"status"`
	Count  int         `json:"count"`
	Page   int         `json:"page"`
	Pages  int         `json:"pages"`
	Now    time.Time   `json:"now"`
	Result interface{} `json:"result"`
}

type Tag struct {
	Title string `json:"title"`
	Level string `json:"level"`
}

type Year struct {
	Year   int      `json:"year"`
	Posts  int      `json:"posts"`
	Months []*Month `json:"months"`
}

type Month struct {
	Month int    `json:"month"`
	Name  string `json:"name"`
	Posts int    `json:"posts"`
}

type Archive struct {
	Tags  interface{} `json:"tags"`
	Years interface{} `json:"years"`
}

func groupContainers(containers []*domain.TagContainter) (map[string]int, map[int64]time.Time, int) {
	tags := make(map[string]int)
	dates := make(map[int64]time.Time)
	for _, tc := range containers {
		dates[tc.Key.ID] = tc.Created
		for _, t := range tc.Tags {
			tags[t] += 1
		}
	}

	return tags, dates, len(containers)
}

func createTags(grouped map[string]int, total int) []*Tag {
	var result []*Tag
	for s, i := range grouped {
		index := int(float64(i) / float64(total) * 10)
		result = append(result, &Tag{
			Title: s,
			Level: tagRanks[index],
		})
	}
	sort.Slice(result, func(i, j int) bool {
		return result[i].Title < result[j].Title
	})
	return result
}

func createArchive(times map[int64]time.Time) []*Year {
	var result []*Year
	byYear := make(map[int]*Year)
	for _, date := range times {
		y := date.Year()
		m := int(date.Month())
		item, ok := byYear[y]
		if !ok {
			item = &Year{Year: y, Months: []*Month{
				{
					Month: m,
					Name:  months[m-1],
				},
			}}
			byYear[y] = item
		}
		item.Posts += 1
		monthAdded := false
		for _, month := range item.Months {
			if month.Month == m {
				month.Posts += 1
				monthAdded = true
				break
			}
		}
		if !monthAdded {
			item.Months = append(item.Months, &Month{
				Month: m,
				Name:  months[m-1],
				Posts: 1,
			})
		}
	}
	for _, year := range byYear {
		result = append(result, year)
		sort.Slice(year.Months, func(i, j int) bool {
			return year.Months[i].Month > year.Months[j].Month
		})
	}

	sort.Slice(result, func(i, j int) bool {
		return result[i].Year > result[j].Year
	})
	return result
}
