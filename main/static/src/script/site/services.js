angular.module('services', []);

angular.module('services').service('SearchService', function() {
  this.search = function(addr, query, callback) {
        service_call("get", addr, query, callback);
  };
});