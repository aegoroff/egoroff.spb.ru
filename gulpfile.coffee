gulp = require('gulp')
$ = require('gulp-load-plugins')()
main_bower_files = require 'main-bower-files'
del = require 'del'
exec = require('child_process').exec
minimist = require 'minimist'


root_dir = '.'
static_dir = "#{root_dir}/static"
paths =
  clean: [
      "#{static_dir}/dst"
      "#{static_dir}/ext"
      "#{static_dir}/min"
    ]

gulp.task 'clean', ->
  del paths.clean

gulp.task 'bower_install', ->
  $.bower()

gulp.task 'ext', gulp.series('bower_install'), ->
  gulp.src main_bower_files(), {base: 'bower_components'}
  .pipe gulp.dest "#{static_dir}/ext"

gulp.task 'copy', ->
  gulp.src main_bower_files(), {base: 'bower_components'}
    .pipe gulp.dest "#{static_dir}/ext"

gulp.task 'fonts', ->
  gulp.src "#{static_dir}/ext/font-awesome/fonts/*"
    .pipe gulp.dest "#{static_dir}/fonts/font-awesome"
  gulp.src "#{static_dir}/ext/bootstrap/fonts/*"
    .pipe gulp.dest "#{static_dir}/fonts/bootstrap"

