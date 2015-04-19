angular.module 'services', []

angular.module('services').service 'SearchService', ['$http', ($http) ->
  @search = (addr, query, callback) ->
      params = query || {}
      params['callback'] = "JSON_CALLBACK"
      for k, v of params
        delete params[k] if not v?
      separator = if addr.search('\\?') >= 0 then '&' else '?'
      $http
        url: "#{addr}#{separator}#{$.param params}"
        method: "JSONP"
        success: (data, status, headers, config) ->
            callback data
            return
        error: (data, status, headers, config) ->
            callback data
            return
      return

  return
]