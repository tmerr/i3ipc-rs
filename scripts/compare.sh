# laziest way in history to check for documentation changes
wget -nv https://i3wm.org/docs/ipc.html
echo === diff ===
diff ipc.html ipc.html.old
echo === end of diff ===
rm ipc.html
