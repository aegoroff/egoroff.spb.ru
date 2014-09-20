$ ->
  $(document).ready ->
      $(".shortDateFormat").each (idx, elem) ->
          formatElementWith(elem, 'LL')
      $(".longDateFormat").each (idx, elem) ->
          formatElementWith(elem, 'LLL')
      $(".date-from-now").each (idx, elem) ->
        if $(elem).is(":input")
            $(elem).val(window.formatDateFromNow($(elem).val()))
          else
            $(elem).text(window.formatDateFromNow($(elem).text()))
