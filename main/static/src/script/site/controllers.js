angular.module('controllers', ['services']);

angular.module('controllers').controller('SearchController', [
'$scope', 'SearchService', function($scope, SearchService) {
  var searchCompleted;
  $scope.dosearch = function(addr, query) {
    NProgress.start();
    SearchService.search(addr, query, searchCompleted);
  };
  searchCompleted = function(r) {
    var i, pagesCount, ref, x;
    NProgress.done();
    $scope.displayResults = true;
    $scope.count = r.queries.request[0].count;
    this.total = parseInt(r.searchInformation.totalResults, 10);
    pagesCount = Math.ceil(this.total / 10);
    if (pagesCount > 1) {
      $scope.pages = [];
      for (x = i = 1, ref = pagesCount; 1 <= ref ? i <= ref : i >= ref; x = 1 <= ref ? ++i : --i) {
        $scope.pages.push({
          title: x,
          link: '#' + x,
          cacheId: x,
          cls: x === 1 ? 'active' : void 0
        });
      }
    }
    $scope.totalResults = r.searchInformation.totalResults;
    $scope.searchTime = r.searchInformation.formattedSearchTime;
    $scope.items = r.items;
  };
}
]);