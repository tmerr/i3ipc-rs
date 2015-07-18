# laziest way in history to check for documentation changes
wget -nv https://i3wm.org/docs/ipc.html
diff ipc.html ipc.html.old
echo === diff ===
rm ipc.html
echo === end of diff ===
