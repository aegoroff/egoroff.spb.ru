angular.module('services', []);

angular.module('services').service('SearchService', ['$http', function($http) {
  this.search = function(addr, query, callback) {
        service_call("get", addr, query, callback);
  };
}
]);