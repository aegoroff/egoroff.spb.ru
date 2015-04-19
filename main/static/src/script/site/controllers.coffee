angular.module 'controllers', ['services']

angular.module('controllers').controller 'SearchController', ['$scope', 'SearchService', ($scope, SearchService) ->
  $scope.dosearch = (addr, query) ->
      NProgress.start()
      SearchService.search addr, query, searchCompleted

  searchCompleted = (r) ->
    NProgress.done()
    $scope.displayResults = true

    $scope.count = r.queries.request[0].count
    @total = parseInt(r.searchInformation.totalResults, 10)
    pagesCount = Math.ceil(@total / 10)
    if pagesCount > 1
      $scope.pages = []
      $scope.pages.push { title: x, link: '#' + x, cacheId: x, cls: 'active' if x == 1 } for x in [1..pagesCount]

    $scope.totalResults  = r.searchInformation.totalResults
    $scope.searchTime  = r.searchInformation.formattedSearchTime
    $scope.items = r.items

    return
]