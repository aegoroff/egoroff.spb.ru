window.SearchModel = ->
  self = this
  @items = ko.mapping.fromJS([])
  @pages = ko.mapping.fromJS([])
  @totalResults = ko.observable('')
  @searchTime = ko.observable('')

  @displayResults = ko.observable(false)
  @count = 0

  mapping = key: (item) ->
    ko.utils.unwrapObservable item.cacheId

  @dosearch = (addr, query) ->
    NProgress.start()
    service_call "get", addr, query, (v,data) ->
        self.searchCompleted(v)

  @searchCompleted = (r) ->
    NProgress.done()
    self.displayResults true

    self.count = r.queries.request[0].count
    @total = parseInt(r.searchInformation.totalResults, 10)
    pagesCount = Math.ceil(@total / self.count)
    @pa = []
    @pa.push { title: x, link: '#' + x, cacheId: x } for x in [1..pagesCount]

    ko.mapping.fromJS @pa, mapping, self.pages

    self.totalResults  r.searchInformation.totalResults
    self.searchTime  r.searchInformation.formattedSearchTime
    ko.mapping.fromJS r.items, mapping, self.items

    return
  return