window.BlogViewModel = ->
  self = this
  @seed = parseInt($("input#seedValue").val(), 10)
  @limit = parseInt($("input#limitValue").val(), 10)
  @offset = parseInt($("input#offsetValue").val(), 10)
  @hideAddButton = false
  @blogPosts = ko.mapping.fromJS([])
  @titleExt = ko.observable('')

  @getPostsUsingQuery = (query, append) ->
    NProgress.start()
    service_call "get", "/api/v2/posts.json", query, (v,data) ->
                                                        self.successfullyRetrievedPosts(data, append)
    return

  @getPosts = ->
    self.getPostsUsingQuery({"offset": self.offset, "limit": self.limit}, true)
    return

  mapping = key: (item) ->
    ko.utils.unwrapObservable item.id

  @successfullyRetrievedPosts = (posts, append) ->
    NProgress.done()
    if append
      unmapped = ko.mapping.toJS(self.blogPosts)
      if unmapped.length > 0
        Array::push.apply unmapped, posts
        ko.mapping.fromJS unmapped, mapping, self.blogPosts
      else
        ko.mapping.fromJS posts, mapping, self.blogPosts
    else
      ko.mapping.fromJS posts, mapping, self.blogPosts
    self.offset += self.limit
    self.limit = self.seed
    self.hideAddButton = posts.length < self.limit
    return

  @postProcessingLogic = (elements) ->
    $(elements).each (idx, elem) ->
      formatElementWith $(elem).find("span.shortDateFormat"), "LL"
      return

    $("button#moreNews").remove()  if @hideAddButton
    return

  return

$ ->
  api_uri = '/api/v2/posts.json'

  removePager = ->
    $("ul.pager").remove()
    $("ul.pagination").remove()

  removeSelection = ->
    $('div.tags > ul > li > a').removeClass('btn btn-default')
    $('div#accordion > div > a').removeClass('active')

  month = $('div#accordion > div > a')
  month.click (event) ->
    href = event.target.hash
    removeSelection()
    $(this).addClass('active')
    vars = href.split('&')
    y = 1
    m = 0
    for v in vars
      pair = v.split('=')
      if pair[0] == 'month'
          m = parseInt(pair[1], 10)
       else
          y = parseInt(pair[1], 10)

    $("dl#blogcontainer").remove()

    q = { "year" : y, "month" : m }
    dmonth = m - 1
    fmt = 'MMMM YYYY'
    # year's posts case
    if m == 0
      fmt = 'YYYY год'
      q = { "year" : y }
      dmonth = 1

    mmt = moment(new Date(y, dmonth, 10))
    mmt.locale(user_lang())
    window.LoadBlog(q, "записи за " + mmt.format(fmt))
    removePager()
  month.button()

  tag = $('div.tags > ul > li > a')
  tag.click (event) ->
    removeSelection()
    $(this).addClass('btn btn-default')
    href = event.target.hash
    $("dl#blogcontainer").remove()
    t = href.split('=')[1]
    txt = 'все посты по метке: ' + t
    window.LoadBlog({ "tag" : t }, txt)
    removePager()
  tag.button()

