window.BlogViewModel = ->
  self = this
  @seed = parseInt($("input#seedValue").val(), 10)
  @limit = parseInt($("input#limitValue").val(), 10)
  @offset = parseInt($("input#offsetValue").val(), 10)
  @hideAddButton = false
  @blogPosts = ko.mapping.fromJS([])
  @getPosts = ->
    service_call "get", "/api/v2/posts.json",
      offset: self.offset
      limit: self.limit
    , (data) ->
      self.successfullyRetrievedPosts data.result, true
      return

    return

  @getPostsUsingQuery = (query, append) ->
    service_call "get", "/api/v2/posts.json", query
    , (data) ->
      self.successfullyRetrievedPosts data.result, append
      return

    return

  mapping = key: (item) ->
    ko.utils.unwrapObservable item.id

  @successfullyRetrievedPosts = (posts, append) ->
    unmapped = ko.mapping.toJS(self.blogPosts)
    if unmapped.length > 0 and append
      Array::push.apply unmapped, posts
      ko.mapping.fromJS unmapped, mapping, self.blogPosts
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