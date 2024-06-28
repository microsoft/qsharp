  function Main() : Unit {
      // this is coming from local deps
      Foo.DependencyA.Foo(); // why does this not work?
      Foo.DependencyA.MagicFunction();
  }
