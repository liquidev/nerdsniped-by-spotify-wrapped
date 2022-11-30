# Nerdsniped by Spotify Wrapped

In brief: I got nerdsniped by Spotify Wrapped.

It always annoyed me that various apps (including Wrapped) count my top tracks by play count, rather
than play time. So using [my huge ListenBrainz dataset](https://listenbrainz.org/user/liquidev/)
I decided to put an end to that, and make **my own Wrapped.** One which counts by minutes listened,
rather than number of times played.

The reason why I wanted this is because tracks have varying length (duh,) and my hypothesis is that
I usually listen to longer tracks less, because they take longer to listen to (duh 2.) So this
method *should* in theory give me a little more accurate data.

Granted, it's not precise to the minute because I sometimes skip tracks, but that happens very
rarely. If I skip then it's usually at the beginning of the track, and then it doesn't get submitted
as a listen to ListenBrainz.

You can find my top 200 [here](top200.txt).

You're free to use this program to create your own top N-hundred. Just know that it's not
particularly user friendly, so much that you will have to edit the source code to change the
username. I cobbled this together in like 2 hours, don't expect it to do wonders.

## How to use

Set the username in main.rs, then:
```
$ cargo run -- --count 200
```
Try to not use counts that are much larger to prevent strain on ListenBrainz / MusicBrainz servers.
It won't give you much more insightful data anyways.

There are two .json files in the repository that configure the program:
- `bad_data.json` - In case there's a fault in your ListenBrainz dataset, and the track did not get
  a proper MBID but has other metadata (artist name, release name, track name,) you can link it to
  an MBID using this.
- `skip.json` - In case there are a bunch of tracks in your ListenBrainz dataset that you'd like to
  exclude from the results, you can list them here. In my case there's a problem with Bogdan
  Raczynski's *boku mo wakaran*, which all got merged to one track because Spotify or MusicBrainz
  did not have enough metadata for ListenBrainz to tell individual tracks apart.
