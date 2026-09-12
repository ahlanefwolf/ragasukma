# ⚡ Skill Command Registry

Daftar perintah cepat dan otomasi sistem yang dapat dijalankan langsung melalui suara, menu popover, atau terminal.
File ini dapat diedit langsung oleh manusia atau dikelola melalui UI Skill Command.

| ID         | Nama Skill         | Deskripsi                          | Command / Script                                     | Kategori   |
| :--------- | :----------------- | :--------------------------------- | :--------------------------------------------------- | :--------- |
| `SYS-000`  | Buka Terminal      | Membuka jendela terminal bawaan    | `foot`                                               | System     |
| `SYS-001`  | System Info        | Cek ringkasan sistem OS dan RAM    | `foot -e sh -c "fastfetch; echo ''; echo 'Tekan Enter...'; read _"` | System     |
| `AUD-001`  | Naikkan Volume     | Tambah volume audio sebesar 5%     | `wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+`          | Audio      |
| `AUD-002`  | Turunkan Volume    | Kurangi volume audio sebesar 5%    | `wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-`          | Audio      |
| `AUD-003`  | Mute / Unmute      | Beralih bisu suara atau nyala      | `wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle`         | Audio      |
| `BRI-001`  | Naikkan Kecerahan  | Tambah / naikkan kecerahan layar monitor atau laptop | `brightnessctl set +10%`                             | Display    |
| `BRI-002`  | Turunkan Kecerahan | Kurangi / redupkan kecerahan layar monitor atau laptop| `brightnessctl set 10%-`                             | Display    |
| `MED-001`  | Play / Pause Media | Putar atau jeda lagu/video         | `playerctl play-pause`                               | Media      |
| `MED-002`  | Lagu Berikutnya    | Pindah ke musik selanjutnya        | `playerctl next`                                     | Media      |
| `MED-003`  | Lagu Sebelumnya    | Kembali ke musik sebelumnya        | `playerctl previous`                                 | Media      |
| `SCR-001`  | Screenshot Layar   | Ambil tangkapan layar area aktif   | `mkdir -p ~/Pictures/Screenshots && grim -g "$(slurp)" ~/Pictures/Screenshots/shot_$(date +%Y%m%d_%H%M%S).png` | Screenshot |
