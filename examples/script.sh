



age=042

test "$age" = "42" && {
    echo "Old enough."
}

test "$age" = "042" && {
    echo "See, this is just a string-wise comparison."
}

amount=12


test $amount -eq 12 && {
    echo "See, this is number-wise comparison."
}

test $amount -eq 012 && {
    echo "See, this is number-wise comparison."
}

gitt() {
    var="$1" && shift && {
        echo "git $var"
    }

    test "clone" = "$1" && shift && {
        test "reposity" = "$1" && shift && {
            echo "git clone repository"
        }
        test "$1" = "" && {
            echo "git clone"
        }
    }
    test "history" = "$1" && shift && {
        echo "git history"
    }
    test "$1" = "" && {
        echo "git"
    }
}
