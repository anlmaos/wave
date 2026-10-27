scalable_dir = join_paths('hicolor', 'scalable', 'apps')

install_data(
  join_paths(scalable_dir, '@0@.png'.format(app_id)),
  install_dir: join_paths(datadir, 'icons', scalable_dir),
)

if profile == 'default'
  install_data(
    join_paths(scalable_dir, '@0@.png'.format(base_id + '.Devel')),
    install_dir: join_paths(datadir, 'icons', scalable_dir),
  )
endif

symbolic_dir = join_paths('hicolor', 'symbolic', 'apps')

install_data(
  join_paths(symbolic_dir, '@0@-symbolic.svg'.format(base_id)),
  install_dir: join_paths(datadir, 'icons', symbolic_dir),
  rename: '@0@-symbolic.svg'.format(app_id)
)
