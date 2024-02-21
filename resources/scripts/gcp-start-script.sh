#!/bin/bash

cat > test.sh <<EOF
#!/bin/bash

echo "It Works!"
EOF

chmod +x test.sh

./test.sh
