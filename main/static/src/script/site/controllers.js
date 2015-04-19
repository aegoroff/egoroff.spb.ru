angular.module('controllers', ['services']);

angular.module('controllers').controller('SearchController', ['$scope', 'SearchService',
  function($scope, SearchService) {
      $scope.dosearch = function(addr, query, params) {
        NProgress.start();
        if(params) {
            for (var attrname in params) {
                query[attrname] = params[attrname];
            }
        }
        SearchService.search(addr, query, searchCompleted);
      };

      function searchCompleted(r) {
        var i, ref, x;
        NProgress.done();
        $scope.displayResults = true;
        $scope.count = r.queries.request[0].count;
        var total = parseInt(r.searchInformation.totalResults, 10);
        var pagesCount = Math.ceil(total / 10);
        if (pagesCount > 1) {
          $scope.pages = [];
          for (x = i = 1, ref = pagesCount; 1 <= ref ? i <= ref : i >= ref; x = 1 <= ref ? ++i : --i) {
            $scope.pages.push({
              title: x,
              link: '#' + x,
              cacheId: x,
              cls: x === 1 ? 'active' : ''
            });
          }
        }
        $scope.totalResults = r.searchInformation.totalResults;
        $scope.searchTime = r.searchInformation.formattedSearchTime;
        $scope.items = r.items;
      }
  }
]);