$ ->
  createDate = (v) ->
    mmt = moment.utc(v)
    mmt.locale('ru')
    timez = jstz.determine().name()
    return mmt.tz(timez)

  formatDateWith = (v, formatSting) ->
    return createDate(v).format(formatSting)

  formatDateFromNow = (v) ->
    return createDate(v).fromNow()

  formatElementWith = (elem, formatSting) ->
    if $(elem).is(":input")
      $(elem).val(formatDateWith($(elem).val(), formatSting))
    else
      $(elem).text(formatDateWith($(elem).text(), formatSting))

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