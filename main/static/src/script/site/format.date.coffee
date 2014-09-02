$ ->
  formatDateFromNow = (v) ->
    return createDate(v).fromNow()

  $(document).ready ->
      $(".shortDateFormat").each (idx, elem) ->
          formatElementWith(elem, 'LL')
      $(".longDateFormat").each (idx, elem) ->
          formatElementWith(elem, 'LLL')
      $(".date-from-now").each (idx, elem) ->
        if $(elem).is(":input")
            $(elem).val(formatDateFromNow($(elem).val()))
          else
            $(elem).text(formatDateFromNow($(elem).text()))
