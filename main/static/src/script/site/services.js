angular.module('services', []);

angular.module('services').service('SearchService', ['$http', function($http) {
  this.search = function(addr, query, callback) {
    var k, params, separator, v;
    params = query || {};
    params['callback'] = "JSON_CALLBACK";
    for (k in params) {
      v = params[k];
      if (v == null) {
        delete params[k];
      }
    }
    separator = addr.search('\\?') >= 0 ? '&' : '?';
    $http({
      url: "" + addr + separator + ($.param(params)),
      method: "JSONP",
      success: function(data, status, headers, config) {
        callback(data);
      },
      error: function(data, status, headers, config) {
        callback(data);
      }
    });
  };
}
]);