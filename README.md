<div align=center>
<img width="777" height="777" alt="animawave banner" src="https://github.com/user-attachments/assets/97bed73e-c6b4-4d14-af60-c097408237ea" />

<h3>Animawave is an internet radio player that provides access to a station database with over 50,000 stations carefully selected for you by AI </h3>

This repository is a fork of [`maunalinux/shortwave`](https://github.com/maunalinux/shortwave) maintained by Anima Labs.

## FAQ
- **Why is it called 'Animawave'?**

    Animawave combines "Anima" (Latin for soul/life) with "wave" (as in radio waves), representing the vitality and energy of internet radio broadcasting.

    If you want to know more about the naming process, you should read this [blog post](https://blogs.gnome.org/tbernard/2019/04/26/naming-your-app/)

- **Why I cannot edit stations anymore?**

    The edit feature is disabled because of vandalism. I cannot change this. [More information here](http://www.radio-browser.info/gui/#/) and [here](https://github.com/segler-alex/radiobrowser-api/issues/39).

- **Is Animawave compatible with Linux phones?**

    Yes! We use the awesome [libadwaita](https://gitlab.gnome.org/GNOME/libadwaita) library to make the interface adaptive.

![Adaptive Screenshot 1](data/screenshots/5.png)
![Adaptive Screenshot 2](data/screenshots/3.png)

- **Which database does Animawave use?**

    [radio-browser.info](http://www.radio-browser.info/gui/#/). It's a community database, everybody can contribute information.

### Building
1. `git clone https://github.com/anlmaos/wave.git`
2. `cd wave`
3. Install dependencies: `rustup default stable` (or nightly) plus system packages `meson ninja pkg-config` and development packages for `sqlite3 openssl dbus-1 glib-2.0 gio-2.0 gtk4 libadwaita-1 shumate-1.0 lcms2 libseccomp gstreamer-1.0 gstreamer-base-1.0 gstreamer-audio-1.0 gstreamer-plugins-base-1.0 gstreamer-plugins-bad-1.0 gstreamer-bad-audio-1.0`.
4. `meson setup build --prefix=/usr` (add `-Dprofile=development` for a development build)
5. `ninja -C build`
6. `sudo ninja -C build install`

Quick start: `meson setup build --profile=development && ninja -C build && ./build/src/animawave`

