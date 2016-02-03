#!/bin/sh
echo "#!/bin/sh\ncargo test -p ethcore" >> ./.git/hooks/pre-push
chmod +x ./.git/hooks/pre-push
