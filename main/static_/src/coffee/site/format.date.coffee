$ ->
  formatDateFromNow = (v) ->
    return window.createDate(v).fromNow()

  $(document).ready ->
      $(".shortDateFormat").each (idx, elem) ->
          window.formatElementWith(elem, 'LL')
      $(".longDateFormat").each (idx, elem) ->
          window.formatElementWith(elem, 'LLL')
      $(".date-from-now").each (idx, elem) ->
        if $(elem).is(":input")
            $(elem).val window.formatDateFromNow $(elem).val()
          else
            $(elem).text window.formatDateFromNow $(elem).text()