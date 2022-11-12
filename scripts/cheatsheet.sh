set -e

CHEATS=()

while read -r line ; do
    HEADING=$(sed -r 's|.+/// syntax\[(.+)\].+|\1|g' <<< "$line")
    LINE=$(sed -r 's|.+/// syntax\[(.+)\]: (.+)|\2|g' <<< "$line")
    CHEATS+=("$HEADING $LINE")
done <<< "$(grep -R "/// syntax" model_script/src/library)"

IFS=$'\n' CHEATS=($(sort <<<"${CHEATS[*]}"))
unset IFS

echo \# Cheat Sheet;
SECTION=""
for key in "${CHEATS[@]}"; do
  NEW_SECTION=$(echo $key | awk '{print $1;}')
  if [ "$NEW_SECTION" != "$SECTION" ];
  then
    SECTION=$NEW_SECTION
    echo
    echo \## $SECTION;
  fi

  REST=$(echo $key | awk '{print substr($0, index($0, " ")+1)}')
  echo - $REST;
done
