$ ->
  $(document).ready ->
      $(".shortDateFormat").each (idx, elem) ->
          formatElementWith(elem, 'LL')
      $(".longDateFormat").each (idx, elem) ->
          formatElementWith(elem, 'LLL')

formatElementWith = (elem, formatSting) ->
  if $(elem).is(":input")
    mmt = moment($(elem).val())
    mmt.locale('ru')
    $(elem).val(mmt.format(formatSting))
  else
    mmt = moment($(elem).text())
    mmt.locale('ru')
    $(elem).text(mmt.format(formatSting))