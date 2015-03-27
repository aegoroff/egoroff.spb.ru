window.SearchModel = ->
  self = this
  @items = ko.mapping.fromJS([])
  @pages = ko.mapping.fromJS([])
  @totalResults = ko.observable('')
  @searchTime = ko.observable('')

  @displayResults = ko.observable(false)
  @count = 0

  @runSearch = (key, cx, params, action_uri) ->
    searchquery = {
      "q" : $( "#q" ).val(),
      "key" : key,
      "cx" : cx
    }
    if params
      for attrname in params
        searchquery[attrname] = params[attrname]

    self.dosearch(action_uri, searchquery)

  @dosearch = (addr, query) ->
    NProgress.start()
    service_call "get", addr, query, (v,data) ->
        self.searchCompleted(v)

  @searchCompleted = (r) ->
    NProgress.done()
    self.displayResults true

    self.count = r.queries.request[0].count
    @total = parseInt(r.searchInformation.totalResults, 10)
    pagesCount = Math.ceil(@total / 10)
    if pagesCount > 1
      @pa = []
      @pa.push { title: x, link: '#' + x, cacheId: x, cls: 'active' if x == 1 } for x in [1..pagesCount]
      ko.mapping.fromJS @pa, {}, self.pages

    self.totalResults  r.searchInformation.totalResults
    self.searchTime  r.searchInformation.formattedSearchTime
    ko.mapping.fromJS r.items, {}, self.items

    return
  return