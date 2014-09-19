window.SearchModel = ->
  self = this
  @items = ko.mapping.fromJS([])
  @totalResults = ko.observable('')
  @searchTime = ko.observable('')
  @displayResults = ko.observable(false)

  mapping = key: (item) ->
    ko.utils.unwrapObservable item.cacheId

  @dosearch = (addr, query) ->
    NProgress.start()
    service_call "get", addr, query, (v,data) ->
        self.searchCompleted(v)

  @searchCompleted = (r) ->
    NProgress.done()
    self.displayResults true
    self.totalResults  r.searchInformation.totalResults
    self.searchTime  r.searchInformation.searchTime
    ko.mapping.fromJS r.items, mapping, self.items

    return
  return